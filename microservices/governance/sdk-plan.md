---
doc_class: SdkPlan
title: SDK Plan (Tenant + Internal Clients)
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + axis-application
deciders: axis-foundry, axis-application, council-architecture
related_artifacts:
  - microservices/governance/contracts/openapi/governance.yaml
  - microservices/governance/contracts/proto/governance.proto
review_cadence: per-version + quarterly
doc_status: published
---

# SDK Plan: governance µservice

## Purpose

The governance µservice ships SDKs for tenant-side + internal-µservice consumers per ADR-0105 `sdk` layer mandate. SDKs close the "industry-standard client library" parity gap vs SonarQube, GHAS, Snyk (each ships SDKs in Java, Python, JS, Go).

## SDK targets

| SDK | Language | Initial release | Maintenance |
|---|---|---|---|
| `oya-governance-lane-runtime-sdk` (Rust) | Rust | M01 (initial; alongside crate scaffolding) | per-PR via Cargo workspace |
| `oya-governance-policy-engine-sdk` (Rust) | Rust | M01 | per-PR |
| `oya-governance-evidence-emitter-sdk` (Rust) | Rust | M01 | per-PR |
| `oya-governance-aggregation-indexer-sdk` (Rust) | Rust | M01 | per-PR |
| `@oyatie/governance-sdk` (TypeScript) | TypeScript | M02 | quarterly minor; per-PR patch |
| `oyatie-governance-sdk` (Python) | Python | M03 | quarterly minor |
| `Oyatie.Governance` (Go) | Go | M04 | quarterly minor |

## Source-of-truth → SDK generation

Generation pipeline:

```text
OpenAPI (contracts/openapi/governance.yaml) ──┐
                                              │
gRPC proto (contracts/proto/governance.proto) ─┼─→ codegen ─→ per-language SDK
                                              │
AsyncAPI (contracts/asyncapi/governance-events.yaml) ─┘
```

- **Rust** SDK: hand-written; thin wrapper around the µservice's REST + gRPC + bus client crates. Lives at `microservices/governance/src/crates/oya-governance-<bc>-sdk/`.
- **TypeScript / Python / Go** SDKs: generated from OpenAPI + proto via `openapi-generator` + `protoc-gen-{ts,python,go}`; published to npm + PyPI + go-modules.

## SDK surface

Per ADR-0105 §"sdk-kernel-only" — SDK depends ONLY on kernel + api layer. No I/O implementation imports.

```rust
// oya-governance-lane-runtime-sdk
pub use oya_governance_lane_runtime_api::*;       // typed I/O contracts
pub use oya_governance_lane_runtime_kernel::*;    // entities + port traits

pub struct GovernanceClient {
    // private state; HTTP/gRPC connection
}

impl GovernanceClient {
    pub fn connect(endpoint: &str, oidc_token: &str) -> Result<Self, Error>;
    pub async fn dispatch_lane(&self, req: DispatchLaneRequest) -> Result<DispatchLaneResponse, Error>;
    pub async fn query_finding(&self, id: &str) -> Result<Finding, Error>;
    pub async fn query_admission_verdict(&self, pr: u64, branch: &str) -> Result<AdmissionVerdict, Error>;
    // ...
}
```

```typescript
// @oyatie/governance-sdk (TypeScript)
import { GovernanceClient } from '@oyatie/governance-sdk';

const client = new GovernanceClient({
  endpoint: 'https://governance.pack-kr.oyatie.dev',
  oidcToken: await fetchToken()
});

const verdict = await client.queryAdmissionVerdict({ prNumber: 123, targetBranch: 'dev' });
```

## Versioning + Compatibility

- SemVer strictly enforced.
- Backwards-incompatible changes → major version bump + 6-month deprecation window per `feedback_no_silent_regression.md`.
- `oya-check-active-artifact-contract` lane refuses contract breaks without ADR.

## Distribution

| SDK | Distribution channel |
|---|---|
| Rust | crates.io OR internal Cargo registry |
| TypeScript | npm; scope `@oyatie` |
| Python | PyPI; package `oyatie-governance-sdk` |
| Go | go-modules; module path `github.com/oyatie/oyatie/sdk/governance` |

## Documentation

- README per SDK at `microservices/governance/src/crates/oya-governance-<bc>-sdk/README.md`.
- API reference auto-generated from contract sources.
- Quick-start tutorial per SDK at `microservices/governance/sdk/<lang>/quickstart.md`.

## Testing

- Each SDK ships with unit tests against mock server fixtures.
- E2E integration test: real µservice + real SDK; in `microservices/governance/tests/e2e/sdk-<lang>.rs` (Rust) or per-language equivalent.

## Authentication patterns

| Pattern | Use case |
|---|---|
| OIDC bearer | Tenant operator + internal engineer |
| SPIFFE workload identity | Internal µservice → governance gRPC |
| Per-µservice service-account token (OpenBao-issued) | CI runner → governance REST |
| JIT short-lived token | External auditor (per `policy/auditor-scope.cedar`) |

## Roadmap

| Release | Date | Content |
|---|---|---|
| 0.1.0 (M01 alpha) | 2026-Q2 | Rust SDKs (4 BC SDKs) |
| 0.2.0 (M02 beta) | 2026-Q3 | + TypeScript SDK |
| 0.3.0 (M03 GA) | 2026-Q4 | + Python SDK; GA stability gate |
| 0.4.0 | 2027-Q1 | + Go SDK |
| 1.0.0 | 2027-Q2 | API stability; semver guarantee |

## Verification

- `cargo doc -p oya-governance-<bc>-sdk --no-deps` exits 0.
- Per-SDK example program in `microservices/governance/tests/e2e/sdk-<lang>.rs` exits 0.
- `oya-check-sdk-kernel-only` lane refuses SDK importing non-kernel/api layers.

## References

- ADR-0105 §"sdk-kernel-only" lane.
- `microservices/governance/contracts/openapi/governance.yaml` + `proto/governance.proto`.
- `microservices/observability/sdk-plan.md` (shape reference).
- OpenAPI Generator — `openapi-generator.tech`.
- `protoc-gen-{ts,python,go}` — `grpc.io/docs/languages/`.
