---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-007-capability-invocation-recorder-stack
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence
acceptance_lanes: [cargo-clippy, cargo-doc, lean-layer-correctness, integration-tests, load-drill-record-invocation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: Capability-invocation-recorder REST stack

## Intent

`oya-foundry-evidence-capability-invocation-recorder-{domain,usecase,api,adapter,rest,sdk}`: synchronous REST receipt + WAL + dead-letter for record_invocation. p99 ≤ 500 ms.

## ChangeSet boundary

6 Rust crates. REST surface bound to OpenAPI contract.

## Concrete File Targets

| Crate | Layer | Notes |
|---|---|---|
| `oya-foundry-evidence-capability-invocation-recorder-domain` | domain | envelope validation rules |
| `oya-foundry-evidence-capability-invocation-recorder-usecase` | usecase | RecordInvocationUsecase: idempotency check + WAL append + pack-builder enqueue |
| `oya-foundry-evidence-capability-invocation-recorder-api` | api | re-exports |
| `oya-foundry-evidence-capability-invocation-recorder-adapter` | adapter | WAL adapter (Postgres + dead-letter cooperation) |
| `oya-foundry-evidence-capability-invocation-recorder-rest` | rest | axum router; OpenAPI conformance |
| `oya-foundry-evidence-capability-invocation-recorder-sdk` | sdk | Rust client; per `sdk-plan.md` |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-capability-invocation-recorder-domain
cargo check -p oya-foundry-evidence-capability-invocation-recorder-usecase
cargo check -p oya-foundry-evidence-capability-invocation-recorder-api
cargo check -p oya-foundry-evidence-capability-invocation-recorder-adapter
cargo check -p oya-foundry-evidence-capability-invocation-recorder-rest
cargo check -p oya-foundry-evidence-capability-invocation-recorder-sdk
cargo nextest run -p oya-foundry-evidence-capability-invocation-recorder-usecase --test record_happy_path
cargo nextest run -p oya-foundry-evidence-capability-invocation-recorder-rest --test openapi_conformance
buck2 build //:quality-lane-registry-authority-check # lane=cedar-tenant-scope --microservice foundry-evidence
oya gate validate load-drill-record-invocation --microservice foundry-evidence
# load-drill verifies p99 ≤ 500 ms sustained at peak target.
```

## Halt Conditions

- p99 record_invocation drill exceeds 500 ms — block; performance regression.
- Idempotency dedup misses a duplicate within 24 h — block (FR-01 contract).
- REST surface diverges from `contracts/openapi/foundry-evidence.yaml` — block.

## Next IP

[`IP-008-eval-evidence-aggregator.md`](IP-008-eval-evidence-aggregator.md)

## References

- `contracts/openapi/foundry-evidence.yaml`.
- ADR-0133 (load-drill claim assertion).

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `evidence`-bounded-context slice for `IP-007: Capability-invocation-recorder REST stack`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: cryptographically sealed evidence packs for invocations, evals, guardrails, and regulator exports. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/intelligence/capabilities/evidence-evidence-pack-build.yaml`, `microservices/intelligence/capabilities/evidence-evidence-query.yaml`, `microservices/intelligence/capabilities/evidence-regulator-export.yaml`, `microservices/intelligence/contracts/openapi/evidence-foundry-evidence.yaml`, and the policy set `microservices/intelligence/policy/evidence-tenant-scope.cedar`, `microservices/intelligence/policy/evidence-regulator-export-scope.cedar`, `microservices/intelligence/policy/evidence-evidence-pack-integrity.md`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `evidence` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/intelligence/manifest.json`; the primary implementation anchor is `crates/oya-intelligence-evidence-domain/src/lib.rs` plus the matching catalog records under `microservices/intelligence/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/intelligence/contracts/openapi/evidence-foundry-evidence.yaml`, `microservices/intelligence/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, and `microservices/intelligence/contracts/proto/evidence-foundry-evidence.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/intelligence/policy/evidence-tenant-scope.cedar`, `microservices/intelligence/policy/evidence-regulator-export-scope.cedar`, `microservices/intelligence/policy/evidence-evidence-pack-integrity.md`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/intelligence/slos/evidence-emit-latency.openslo.yaml`, `microservices/intelligence/slos/evidence-chain-integrity-correctness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/intelligence/capabilities/evidence-evidence-pack-build.yaml`, `microservices/intelligence/capabilities/evidence-evidence-query.yaml`, `microservices/intelligence/capabilities/evidence-regulator-export.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/intelligence/PRD.md` and the `evidence` row in `microservices/intelligence/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/intelligence/catalog/`, `microservices/intelligence/contracts/`, `microservices/intelligence/policy/`, or `microservices/intelligence/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-intelligence-evidence-domain/src/lib.rs`.
- Contract parity for `microservices/intelligence/contracts/openapi/evidence-foundry-evidence.yaml` and `microservices/intelligence/contracts/proto/evidence-foundry-evidence.proto` when DTOs or handlers change.
- Policy resolution against `microservices/intelligence/policy/evidence-tenant-scope.cedar`, `microservices/intelligence/policy/evidence-regulator-export-scope.cedar`, `microservices/intelligence/policy/evidence-evidence-pack-integrity.md`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/intelligence/slos/evidence-emit-latency.openslo.yaml`, `microservices/intelligence/slos/evidence-chain-integrity-correctness.openslo.yaml`; no acceptance by line count alone.
- `buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/intelligence/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/intelligence/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| Palantir AIP audit evidence and ServiceNow GRC evidence export | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
