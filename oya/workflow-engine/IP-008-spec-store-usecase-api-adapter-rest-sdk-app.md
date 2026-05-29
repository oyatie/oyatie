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
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-workflow-spec-signature-verification, oya-governance-spec-construct-conformance]
---

# IP-008: append-only spec-store usecase, API, Postgres adapter, REST, SDK, and app

## §A Problem

Workflow specs are the executable contract between Studio, SDK consumers, and the runtime. `microservices/workflow-engine/policy/spec-integrity.md` already defines the canonical `(tenant_id, spec_id, version_sha)` identity model, Ed25519 signing scheme, lifecycle states, forbidden constructs, and replay safety requirements. The short stamped IP did not bind those rules to concrete usecases, persistence, REST routes, or SDK behavior.

This IP closes the gap where a tenant could submit a spec, receive a version, start a run, and later discover that replay used a mutated or unsigned body. For workflow-engine, spec storage is not CRUD; it is append-only executable provenance.

## §B Approach

Build the remaining `spec-store` layers declared in `microservices/workflow-engine/manifest.json`: usecase, api, adapter, adapter-postgres, rest, sdk, and app. The usecase layer owns `SpecSubmissionOrchestrator` and `SpecLifecycleOrchestrator`. The domain work from IP-002 canonicalizes and verifies signatures; this IP persists the result, exposes `/specs`, `/specs/{spec_id}/versions/{version_sha}`, and `/specs/{spec_id}/lifecycle`, and gives tenants an SDK that cannot bypass signature or lifecycle gates.

Persistence is append-only: `spec_versions` receives immutable rows; lifecycle changes are recorded in a separate lifecycle ledger. Run-starts can load only `published` versions, while replay can load deprecated or retired versions for audit.

## §C Deliverables

| Artifact | Action | Substance requirement |
|---|---|---|
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-usecase/src/submission.rs` | create | canonicalize, verify signature, reject forbidden constructs, insert immutable version |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-usecase/src/lifecycle.rs` | create | deprecate/retire transitions with two-person rule and reason capture |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-api/src/types.rs` | create | typed `WorkflowSpecSubmission`, `WorkflowSpec`, `SpecLifecycleTransition` aligned to OpenAPI/proto |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-adapter-postgres/migrations/V1__spec_versions_schema.sql` | create | append-only `spec_versions`, lifecycle ledger, tenant/version indexes, immutability trigger |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-rest/src/routes.rs` | create | handlers for OpenAPI `submitWorkflowSpec`, `listWorkflowSpecs`, `getWorkflowSpecVersion`, `transitionSpecLifecycle` |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-sdk/src/client.rs` | create | tenant SDK wrapper that signs/canonicalizes before submit and handles idempotent resubmit |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-app/src/main.rs` | create | composition root for REST, Postgres pool, OpenBao key resolver, Cedar bundle, metrics |
| `microservices/workflow-engine/catalog/oya-workflow-engine-spec-store-*.yaml` | update/create | rows for all seven remaining crates |

## §D Implementation

1. Implement `SpecSubmissionOrchestrator::submit` so it derives canonical JSON, computes `version_sha`, verifies Ed25519 signer identity through OpenBao public-key material, and refuses `policy/spec-integrity.md` forbidden constructs.
2. Implement idempotent insert behavior: the same `(tenant_id, spec_id, version_sha)` returns the existing row; any different body under the same `version_sha` is a tamper error.
3. Add the Postgres migration with append-only trigger on `spec_versions` and a mutable-only lifecycle ledger that captures actor, reason, previous state, target state, and audit id.
4. Implement lifecycle rules: `published -> deprecated -> retired`; direct `published -> retired` requires explicit two-person signatures matching OpenAPI `two_person_signature`.
5. Build REST handlers from the existing OpenAPI routes and proto conversions from `SpecStore` RPCs; client-supplied `tenant_id` remains impossible.
6. Add SDK signing helpers and canonicalization preflight so Studio and tenant code see the same `version_sha` the server will compute.
7. Emit audit-chain events for submit, lifecycle transition, revocation pause, and tamper refusal, with metrics for revocation propagation lag.

## §E Acceptance

- `cargo nextest run -p oya-workflow-engine-spec-store-usecase --all-features`
- `cargo nextest run -p oya-workflow-engine-spec-store-adapter-postgres --all-features`
- `cargo nextest run -p oya-workflow-engine-spec-store-rest --all-features`
- `cargo nextest run -p oya-workflow-engine-spec-store-sdk --all-features`
- `cargo run -p oya-dev-cli -- gate validate workflow-spec-signature-verification --crate oya-workflow-engine-spec-store-usecase`
- `cargo run -p oya-dev-cli -- gate validate spec-construct-conformance --crate oya-workflow-engine-spec-store-usecase`
- Required tests: `submit_signature_verified_on_write`, `load_signature_verified_on_read`, `resubmit_same_spec_is_idempotent`, `tampered_row_refused`, `retire_requires_two_person_signature`, and `deprecated_spec_replay_allowed_new_run_refused`.

## §F Evidence

- `microservices/workflow-engine/policy/spec-integrity.md` is the canonical spec identity, signing, lifecycle, and forbidden-construct contract.
- `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml` defines `/specs` submission, list, version read, and lifecycle routes.
- `microservices/workflow-engine/contracts/proto/workflow-engine.proto` defines `SpecStore` RPCs and `SpecLifecycle`.
- `microservices/workflow-engine/runbooks/spec-rollback.md` is the operator rollback anchor for lifecycle mistakes.
- `microservices/workflow-engine/threat-model.md` is cited by spec-integrity for tampering and replay-window risks.

## §G Counterparts

| Counterpart | Relevant behavior | This IP closes |
|---|---|---|
| Temporal | workflow definitions are versioned and replay must honor old code paths | immutable `version_sha` plus deprecated/retired replay rules |
| AWS Step Functions | state machine definitions are versionable and execution-bound | run-start pins a published spec version and replay loads that same version |
| Camunda 8 | BPMN process definitions are deployed and versioned before execution | spec lifecycle ledger separates executable definition changes from run state |
| n8n | workflow JSON can drift unless deployment discipline is enforced | Ed25519 signature and append-only storage prevent silent mutable specs |

## Next IP

[`IP-009-execution-engine-rest-worker-sdk-app.md`](IP-009-execution-engine-rest-worker-sdk-app.md)

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.
