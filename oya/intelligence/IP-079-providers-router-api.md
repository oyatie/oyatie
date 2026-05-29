---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-004-router-api
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: oya-foundry-providers-router-api

## Intent

Protocol-neutral typed contracts (DTOs) shared by REST + gRPC + SDK + adapter layers. Mirrors OpenAPI + proto shapes but is the canonical Rust source of truth.

## ChangeSet boundary

One new crate `microservices/intelligence/src/crates/oya-foundry-providers-router-api/`. Depends on `oya-foundry-providers-router-kernel`.

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/dto/router.rs` | create — `RoutingRequestDto`, `RouterDecisionDto` |
| `.../src/dto/invoke.rs` | create — `InvokeRequestDto`, `InvokeResponseDto`, `InvokeStreamChunkDto` |
| `.../src/dto/health.rs` | create — `ProviderHealthSnapshotDto` |
| `.../src/dto/config.rs` | create — `TenantProviderConfigDto` |
| `.../src/dto/disclosure.rs` | create — `EuAiActDisclosureDto` |
| `.../src/conv.rs` | create — `From` impls between DTOs and kernel entities |

## Test Plan

| Test | Verifies |
|---|---|
| DTO ⇄ entity roundtrip | parity with kernel |
| DTO serde JSON | matches OpenAPI schema |
| DTO serde proto | matches proto schema |
| DTO does not include credential bytes | grep regex sweep |

## Acceptance Gates

Standard + `contract-conformance` lane (DTOs match `contracts/openapi/provider-router.yaml` + `contracts/proto/provider-invoke.proto`).

## Next IP

[`IP-005-router-adapter.md`](IP-005-router-adapter.md)

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `providers`-bounded-context slice for `IP-004: oya-foundry-providers-router-api`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: tenant-scoped provider routing with OpenBao credential isolation and API/subscription adapters. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/intelligence/capabilities/providers-provider-route.yaml`, `microservices/intelligence/capabilities/providers-provider-invoke.yaml`, `microservices/intelligence/capabilities/providers-credential-resolve.yaml`, `microservices/intelligence/contracts/openapi/providers-provider-router.yaml`, and the policy set `microservices/intelligence/policy/providers-provider-router-tenant-scope.cedar`, `microservices/intelligence/policy/providers-openbao-credential.cedar`, `microservices/intelligence/policy/providers-credential-isolation.md`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `providers` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/intelligence/manifest.json`; the primary implementation anchor is `microservices/intelligence/catalog/oya-foundry-providers-router-*.yaml` plus the matching catalog records under `microservices/intelligence/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/intelligence/contracts/openapi/providers-provider-router.yaml`, `microservices/intelligence/contracts/asyncapi/providers-provider-events.yaml`, and `microservices/intelligence/contracts/proto/providers-provider-invoke.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/intelligence/policy/providers-provider-router-tenant-scope.cedar`, `microservices/intelligence/policy/providers-openbao-credential.cedar`, `microservices/intelligence/policy/providers-credential-isolation.md`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/intelligence/slos/providers-availability-openai.openslo.yaml`, `microservices/intelligence/slos/providers-availability-google.openslo.yaml`, `microservices/intelligence/slos/providers-circuit-breaker-correctness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/intelligence/capabilities/providers-provider-route.yaml`, `microservices/intelligence/capabilities/providers-provider-invoke.yaml`, `microservices/intelligence/capabilities/providers-credential-resolve.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/intelligence/PRD.md` and the `providers` row in `microservices/intelligence/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/intelligence/catalog/`, `microservices/intelligence/contracts/`, `microservices/intelligence/policy/`, or `microservices/intelligence/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `microservices/intelligence/catalog/oya-foundry-providers-router-*.yaml`.
- Contract parity for `microservices/intelligence/contracts/openapi/providers-provider-router.yaml` and `microservices/intelligence/contracts/proto/providers-provider-invoke.proto` when DTOs or handlers change.
- Policy resolution against `microservices/intelligence/policy/providers-provider-router-tenant-scope.cedar`, `microservices/intelligence/policy/providers-openbao-credential.cedar`, `microservices/intelligence/policy/providers-credential-isolation.md`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/intelligence/slos/providers-availability-openai.openslo.yaml`, `microservices/intelligence/slos/providers-availability-google.openslo.yaml`, `microservices/intelligence/slos/providers-circuit-breaker-correctness.openslo.yaml`; no acceptance by line count alone.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/intelligence/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/intelligence/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| LiteLLM, OpenRouter, Anthropic Console, OpenAI API, and Vertex Model Garden | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
