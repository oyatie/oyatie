---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-012-adapter-in-house
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, credential-isolation, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: oya-foundry-providers-adapter-in-house

## Intent

Adapter for oyatie-trained in-house models served via vLLM/TGI co-located endpoint per ADR-0026. Mesh-internal mTLS (no public egress). Implements ProviderInvoker port with canonical kernel shape so the router treats in-house identically.

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/transport.rs` | create — mesh-internal mTLS to in-house vLLM/TGI fleet |
| `.../src/request_builder.rs` | create — kernel → OpenAI-compatible vLLM JSON |
| `.../src/response_parser.rs` | create — vLLM response → kernel canonical |
| `.../src/envelope.rs` | create — BLAKE3 + Ed25519 (same as hosted adapters) |
| `.../src/endpoint_discovery.rs` | create — service-mesh discovery of in-house pool |
| `.../src/canary_cohort.rs` | create — per-tenant cohort weighting (1/10/50/100%) |

## Credentials

In-house adapter does NOT use vendor API keys; mesh mTLS establishes pod-to-pod identity. The CredentialResolver port returns a sentinel `ResolvedCredential::InHouse` value carrying only the SPIFFE peer-identity binding (no secret bytes).

## Test Plan

| Test | Verifies |
|---|---|
| `test_request_builder_matches_vllm_openai_compat_schema` | spec |
| `test_response_parser_canonical_shape` | normalisation |
| `tests/integration/in_house_no_external_egress.rs` | no external network connection during call |
| `tests/integration/in_house_canary_cohort_ramp` | cohort weighting |
| `tests/integration/in_house_burn_rate_demote` | T-04 (quality regression demote) |
| `test_envelope_signed_identically_to_hosted` | parity |

## Acceptance Gates

Standard + `credential-isolation` (sentinel-shaped only).

## Next IP

[`IP-013-adapter-openbao.md`](IP-013-adapter-openbao.md)

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `providers`-bounded-context slice for `IP-012: oya-foundry-providers-adapter-in-house`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: tenant-scoped provider routing with OpenBao credential isolation and API/subscription adapters. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/foundry/capabilities/providers-provider-route.yaml`, `microservices/foundry/capabilities/providers-provider-invoke.yaml`, `microservices/foundry/capabilities/providers-credential-resolve.yaml`, `microservices/foundry/contracts/openapi/providers-provider-router.yaml`, and the policy set `microservices/foundry/policy/providers-provider-router-tenant-scope.cedar`, `microservices/foundry/policy/providers-openbao-credential.cedar`, `microservices/foundry/policy/providers-credential-isolation.md`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `providers` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/foundry/manifest.json`; the primary implementation anchor is `crates/oya-intelligence-provider-pool-kernel/src/lib.rs` plus the matching catalog records under `microservices/foundry/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/foundry/contracts/openapi/providers-provider-router.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, and `microservices/foundry/contracts/proto/providers-provider-invoke.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/foundry/policy/providers-provider-router-tenant-scope.cedar`, `microservices/foundry/policy/providers-openbao-credential.cedar`, `microservices/foundry/policy/providers-credential-isolation.md`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/foundry/slos/providers-availability-openai.openslo.yaml`, `microservices/foundry/slos/providers-availability-google.openslo.yaml`, `microservices/foundry/slos/providers-circuit-breaker-correctness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/foundry/capabilities/providers-provider-route.yaml`, `microservices/foundry/capabilities/providers-provider-invoke.yaml`, `microservices/foundry/capabilities/providers-credential-resolve.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/foundry/PRD.md` and the `providers` row in `microservices/foundry/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/foundry/catalog/`, `microservices/foundry/contracts/`, `microservices/foundry/policy/`, or `microservices/foundry/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-intelligence-provider-pool-kernel/src/lib.rs`.
- Contract parity for `microservices/foundry/contracts/openapi/providers-provider-router.yaml` and `microservices/foundry/contracts/proto/providers-provider-invoke.proto` when DTOs or handlers change.
- Policy resolution against `microservices/foundry/policy/providers-provider-router-tenant-scope.cedar`, `microservices/foundry/policy/providers-openbao-credential.cedar`, `microservices/foundry/policy/providers-credential-isolation.md`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/foundry/slos/providers-availability-openai.openslo.yaml`, `microservices/foundry/slos/providers-availability-google.openslo.yaml`, `microservices/foundry/slos/providers-circuit-breaker-correctness.openslo.yaml`; no acceptance by line count alone.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/foundry/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/foundry/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| LiteLLM, OpenRouter, Anthropic Console, OpenAI API, and Vertex Model Garden | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
