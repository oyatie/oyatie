---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-008-spec-store-usecase-api-adapter-rest-sdk-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-workflow-spec-signature-verification]
---

# IP-008: spec-store remaining layers (usecase + api + adapter + adapter-postgres + rest + sdk + app)

## Intent

Complete spec-store BC. Adds Postgres-backed spec storage; rest surface for spec submission + lifecycle; SDK for tenant spec submission; app composition root.

## ChangeSet boundary

7 new crates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-workflow-engine-spec-store-usecase/{...}` | create | SpecSubmissionOrchestrator, SpecLifecycleOrchestrator |
| `src/crates/oya-workflow-engine-spec-store-api/{...}` | create | typed I/O |
| `src/crates/oya-workflow-engine-spec-store-adapter/{...}` | create | protocol-neutral impls |
| `src/crates/oya-workflow-engine-spec-store-adapter-postgres/{Cargo.toml,src/lib.rs,migrations/V1__spec_versions_schema.sql}` | create | append-only `spec_versions` table; INSERT-only Postgres trigger |
| `src/crates/oya-workflow-engine-spec-store-rest/{...}` | create | HTTP routes |
| `src/crates/oya-workflow-engine-spec-store-sdk/{...}` | create | Tenant SDK |
| `src/crates/oya-workflow-engine-spec-store-app/{...}` | create | Composition root |
| `microservices/workflow-engine/catalog/oya-workflow-engine-spec-store-*.yaml` | create | 7 catalog rows |

## Acceptance Gates

```bash
cargo nextest run -p oya-workflow-engine-spec-store-usecase --all-features
cargo nextest run -p oya-workflow-engine-spec-store-adapter-postgres --all-features
cargo run -p oya-dev-cli -- gate validate workflow-spec-signature-verification --crate oya-workflow-engine-spec-store-usecase
cargo run -p oya-dev-cli -- gate validate spec-construct-conformance --crate oya-workflow-engine-spec-store-usecase
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_spec_submission_signature_verified` | signature check exercised on every read |
| `test_spec_submission_idempotent` | same body + same metadata → same version_sha; no duplicate row |
| `test_spec_lifecycle_transition_2person_rule` | retired transition refuses single-signer |
| `test_spec_tampering_at_read_time_detected` | tampered Postgres row refused at read |

## Next IP

[`IP-009-execution-engine-rest-worker-sdk-app.md`](IP-009-execution-engine-rest-worker-sdk-app.md)

## References

- PRD FR-01, FR-02
- `policy/spec-integrity.md`
- `contracts/openapi/workflow-engine.yaml`
