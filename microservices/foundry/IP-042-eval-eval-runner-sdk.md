---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-012-eval-runner-sdk
status: pending
execution_unit: ChangeSet
owner: axis-foundry + axis-developer-experience
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: oya-foundry-eval-eval-runner-sdk

## Intent

Rust client SDK for capability owners + tenant operators; closes the OpenAI-Evals / Anthropic-evals / LangSmith / Patronus / Braintrust SDK gap. TS + Python bindings via wasm-bindgen + PyO3 (M02).

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-eval-eval-runner-sdk/`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-eval-eval-runner-sdk/Cargo.toml` | create — `reqwest`, `serde`, `oya-foundry-eval-eval-runner-api` |
| `src/crates/oya-foundry-eval-eval-runner-sdk/src/lib.rs` | create |
| `src/crates/oya-foundry-eval-eval-runner-sdk/src/client.rs` | create — Client + builder |
| `src/crates/oya-foundry-eval-eval-runner-sdk/src/operations.rs` | create — per-endpoint methods |
| `src/crates/oya-foundry-eval-eval-runner-sdk/src/error.rs` | create — typed errors + retry helpers |
| `src/crates/oya-foundry-eval-eval-runner-sdk/src/observability.rs` | create — OTel spans |
| `catalog/oya-foundry-eval-eval-runner-sdk.yaml` | create |

## Test Plan

90% line: client-side roundtrip against `oya-foundry-eval-eval-runner-rest` (workspace integration test).

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `eval`-bounded-context slice for `IP-012: oya-foundry-eval-eval-runner-sdk`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: replayable eval execution with baseline output storage and parity comparison. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/contracts/openapi/eval-eval-runner.yaml`, and the policy set `microservices/foundry/policy/eval-tenant-scope.cedar`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-ci-scope.cedar`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `eval` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/foundry/manifest.json`; the primary implementation anchor is `microservices/foundry/catalog/oya-foundry-eval-eval-runner-*.yaml` plus the matching catalog records under `microservices/foundry/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/foundry/contracts/openapi/eval-eval-runner.yaml`, `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, and `microservices/foundry/contracts/proto/eval-eval_runner.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/foundry/policy/eval-tenant-scope.cedar`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-ci-scope.cedar`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/foundry/PRD.md` and the `eval` row in `microservices/foundry/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/foundry/catalog/`, `microservices/foundry/contracts/`, `microservices/foundry/policy/`, or `microservices/foundry/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `microservices/foundry/catalog/oya-foundry-eval-eval-runner-*.yaml`.
- Contract parity for `microservices/foundry/contracts/openapi/eval-eval-runner.yaml` and `microservices/foundry/contracts/proto/eval-eval_runner.proto` when DTOs or handlers change.
- Policy resolution against `microservices/foundry/policy/eval-tenant-scope.cedar`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-ci-scope.cedar`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`; no acceptance by line count alone.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/foundry/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/foundry/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| LangSmith evals, OpenAI Evals, and Google Vertex AI evaluation jobs | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
