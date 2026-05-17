---
doc_class: SdkPlan
title: SDK Plan (Rust + TS + Python clients)
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-ontology + dx-sdk
deciders: axis-ontology, council-architecture, dx-sdk
related_adrs: [ADR-0056, ADR-0059, ADR-0105, ADR-0106, ADR-0131]
related_artifacts:
  - microservices/ontology/contracts/openapi/ontology.yaml
  - microservices/ontology/contracts/proto/ontology.proto
  - microservices/ontology/contracts/asyncapi/ontology-events.yaml
review_cadence: per major version + on every breaking REST or gRPC change
doc_status: published
---

# SDK Plan (ontology µservice)

## Purpose

Define the multi-language SDK strategy for the Ontology µservice. Closes the industry-standard typed-entity SDK gap (Palantir Foundry SDK parity, Salesforce SOAP/REST SDK parity, Notion API client parity) so that every µservice + product author + external consumer reads/writes Ontology with a consistent type-safe interface.

## SDK Languages — Tier-A (M02b launch)

| Language | Status | Source format | Publisher | Why |
|---|---|---|---|---|
| **Rust** | shipped | Hand-authored from kernel + auto-generated from OpenAPI / Proto | crates.io (private registry first; oyatie/ once GA) | First-class internal language; every Layer-B µservice consumes through this |

Rust is the **first-class** SDK — kernel types (`ObjectTypeSchema`, `ObjectInstance`, `LinkInstance`, `ActionInvocationReceipt`, `FunctionResult`, `MerkleSealRecord`) live in `oya-ontology-*-kernel` crates and the SDK re-exports them with type-safe builder patterns.

## SDK Languages — Tier-B (post-M02b)

| Language | Status | Source format | Publisher | Target |
|---|---|---|---|---|
| **TypeScript** | M03 | Auto-generated from OpenAPI 3.2 + AsyncAPI 3.0 + Proto | npm (`@oyatie/ontology-sdk`) | Workflow Studio + tenant-facing apps + Node.js workload µservices |
| **Python** | M03 | Auto-generated from OpenAPI + Proto + Pydantic models | PyPI (`oyatie-ontology-sdk`) | Data-science + LLM agent harness + Python workload µservices |
| **Go** | M04 | Auto-generated from Proto | private Go module path → public after GA | Go-based external consumers; Foundry-like external integration |
| **Java/Kotlin** | M04 | Auto-generated from Proto via grpc-java + OpenAPI Generator | Maven Central post-GA | JVM-based external consumers |

## Rust SDK (Tier-A; first-class)

### Crate layout

```
oya-ontology-sdk             — re-exports kernel types + REST/gRPC clients
  ├── oya-ontology-object-type-registry-sdk
  ├── oya-ontology-link-type-registry-sdk
  ├── oya-ontology-action-type-registry-sdk
  ├── oya-ontology-function-type-registry-sdk
  ├── oya-ontology-entity-store-sdk
  ├── oya-ontology-link-store-sdk
  ├── oya-ontology-function-engine-sdk
  ├── oya-ontology-action-engine-sdk
  ├── oya-ontology-query-engine-sdk
  ├── oya-ontology-agent-gateway-sdk
  ├── oya-ontology-audit-chain-sdk
```

### Public API surface (selected)

```rust
use oya_ontology_sdk::OntologyClient;
use oya_ontology_object_type_registry_kernel::{ObjectTypeSchema, PillarKind, PropertyTier};

let client = OntologyClient::builder()
    .pack("kr")
    .tenant_id(tenant_id)
    .oidc_token(token)
    .build()?;

// Register an Object Type (governance-controlled; usually PR-only)
let schema = ObjectTypeSchema::builder()
    .name("Patient")
    .pillar_kind(PillarKind::Person)
    .property("medical_record_number", PropertyTier::Tier1Sensitive, "PHI", PropertyType::Scalar)
    .property("admission_date", PropertyTier::Tier2Restricted, "BEHAVIORAL_TENANT_PRODUCT", PropertyType::Scalar)
    .build()?;

client.object_type_registry().register(schema).await?;

// Write an Object Type instance
let patient = client.entity_store()
    .create_object_instance("Patient")
    .property("medical_record_number", "MR-12345")
    .property("admission_date", "2026-05-17")
    .idempotency_key("admission-event-uuid")
    .send()
    .await?;

// Read a Function (typed projection)
let active_patients: Vec<Patient> = client.function_engine()
    .evaluate::<PatientsByStatus>(PatientsByStatusArgs {
        status: "active",
    })
    .await?;

// Invoke an Action (Cedar-gated)
let receipt = client.action_engine()
    .invoke::<DischargePatientAction>(DischargePatientArgs {
        patient_id: patient.object_id,
    })
    .idempotency_key(uuid::Uuid::new_v4().to_string())
    .send()
    .await?;

// Receipt carries audit_chain_ref + Cedar decision ref
assert!(receipt.audit_chain_ref.starts_with("merkle:"));
```

### Type-safety + codegen

- `ObjectTypeSchema` macros generate strongly-typed Rust structs at compile time when the schema is `#[ontology_object_type(...)]`-annotated.
- `Function` calls statically check argument/result shapes against the Function Type schema at build time (cargo-deny-style).
- `Action` calls statically check Cedar autonomy_tier requirements.

### Retry + reliability

- Exponential backoff with jitter (Tower middleware).
- Per-tenant rate limit awareness (HTTP 429 → exponential backoff + Retry-After header).
- Idempotency key required on writes by default; SDK refuses POST without key.
- Circuit breaker per-pack endpoint.

### Telemetry

- OpenTelemetry traces auto-injected on every SDK call.
- Per-Function + per-Action metrics: `oya_ontology_sdk_call_duration_seconds`, `oya_ontology_sdk_call_total{outcome}`.

### Authentication

- OIDC bearer token (standard);
- agent JWT (for agent-gateway clients);
- per-µservice SPIFFE identity (for in-cluster gRPC).

## TypeScript SDK (Tier-B; M03)

Auto-generated from:
- OpenAPI 3.2 → axios + typed interfaces.
- Proto → connect-es / protobuf-ts.
- AsyncAPI 3.0 → typed event subscribers via Kafka.js.

Public API mirrors Rust SDK; supports:
- Async/await idioms.
- Strict TypeScript types (no `any`).
- React-friendly hooks (`useOntologyObject`, `useFunctionEvaluation`).
- Tree-shaking; minimal bundle size.

## Python SDK (Tier-B; M03)

Auto-generated from:
- OpenAPI 3.2 → httpx + Pydantic v2 models.
- Proto → grpcio-tools + betterproto.
- AsyncAPI 3.0 → aiokafka subscribers.

Public API mirrors Rust SDK; supports:
- Sync + async modes.
- Pandas DataFrame conversion helpers (for analytics use cases).
- Type checking via mypy + pyright.

## SDK Distribution + Versioning

| Channel | Tier-A (Rust) | Tier-B (TS / Python) |
|---|---|---|
| Pre-GA | Private registry / GitHub Packages | Private npm / private PyPI |
| GA | crates.io | npm + PyPI |
| Semver | Major version per breaking REST/gRPC change; minor per Function/Action addition; patch per bug fix | Same |
| Deprecation policy | Per Bominal ADR-0149 (inherited); ≥ 90 days warning + sunset | Same |
| Breaking change | LEAN lane `oya-foundry-fitness-api-semver` refuses any incompatible change | Same |

## SDK Generation Toolchain

| Tool | Purpose | Configuration |
|---|---|---|
| `openapi-generator-cli` | OpenAPI → TypeScript/Python/Go/Java | `openapi-generator-cli generate -i contracts/openapi/ontology.yaml -g rust -o sdk/rust/` |
| `tonic-build` | Proto → Rust gRPC client | build.rs in oya-ontology-sdk |
| `prost-build` | Proto → Rust messages | build.rs |
| `connect-es` | Proto → TypeScript gRPC client | npm script |
| `betterproto` | Proto → Python | poetry script |
| `protoc-gen-go-grpc` | Proto → Go gRPC | Make target |

## Code Generation Pipeline

Every SDK is regenerated on:
1. OpenAPI 3.2 spec change → re-run codegen + cargo build for Rust + npm pack for TS.
2. Proto change → re-run codegen + verify compilation across all Tier-A + Tier-B.
3. AsyncAPI 3.0 change → regenerate event subscriber types.

CI lane `oya-foundry-fitness-sdk-regen-conformance` ensures generated code matches the spec; PRs with stale generated code fail the lane.

## Testing

| Test type | Coverage |
|---|---|
| Unit tests per SDK function | ≥ 1 per public method (happy + auth-fail + tenant-mismatch + rate-limit) |
| Integration tests against rest crate | ≥ 2 cross-route flows per SDK; SDK + REST tested end-to-end |
| Contract tests | OpenAPI spec → SDK match; LEAN lane validates |
| Property tests | Object Type round-trip: write → read returns identical; Function projection round-trip |
| Compatibility tests | Old SDK against new REST surface; new SDK against old REST surface (per Bominal ADR-0149) |

## Documentation

| Doc | Channel |
|---|---|
| Rust SDK rustdoc | `docs.rs/oya-ontology-sdk` (post-GA) |
| OpenAPI spec | `https://ontology-kr.oyatie.dev/api/v1/openapi.yaml` |
| TypeScript declaration | `npm` registry + GitHub Packages |
| Python type stubs | bundled in PyPI package |
| Quickstart guide (per language) | `docs/sdk/<lang>/quickstart.md` |
| Cookbook (common patterns) | `docs/sdk/cookbook.md` |
| Migration guides (major version) | `docs/sdk/migrations/<from>-<to>.md` |

## Adoption Plan

1. **M02b launch (XS tier)**: Rust SDK consumed by every Layer-B µservice in this codebase; tenants use REST/gRPC directly.
2. **M03**: TS + Python SDKs ship; first external tenants integrate.
3. **M04**: Go + Java SDKs; large-enterprise tenants.
4. **Continuous**: SDK competitive parity check vs Palantir Foundry SDK + Salesforce SDK + Notion API client + OpenAI Tools.

## References

- `microservices/ontology/contracts/openapi/ontology.yaml`.
- `microservices/ontology/contracts/proto/ontology.proto`.
- `microservices/ontology/contracts/asyncapi/ontology-events.yaml`.
- ADR-0056 (BNF v4.1).
- ADR-0059 (Workflow + Ontology adapter layer).
- ADR-0105 (13-layer enum).
- ADR-0106 (Bominal — Ontology architecture).
- ADR-0131 (per-microservice flat layout).
- Bominal ADR-0149 (schema evolution; inherited).
- OpenAPI Generator — `openapi-generator.tech`.
- tonic — `docs.rs/tonic`.
- Palantir Foundry SDK reference — `palantir.com/docs/foundry/sdk/`.
