---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-014-router-rest-worker-app
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, integration-test]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: router-rest + router-worker + router-app (composition root)

## Intent

Three crates: the REST surface (`-rest`), the health-monitor + cost-roll-up worker (`-worker`), and the composition-root binary (`-app`) that wires usecase + adapter + every per-vendor adapter into a runnable service.

## File Targets

### `oya-foundry-providers-router-rest`

| Path | Action |
|---|---|
| `.../Cargo.toml` | create — `axum`, `tower`, OIDC + Cedar middleware deps |
| `.../src/lib.rs` | create |
| `.../src/handlers/decide.rs` | create — POST /router/decide |
| `.../src/handlers/invoke.rs` | create — POST /router/invoke |
| `.../src/handlers/health.rs` | create — GET /providers/health |
| `.../src/handlers/capabilities.rs` | create — GET /providers/capabilities |
| `.../src/handlers/tenant_config.rs` | create — GET/PUT /providers/config/{tenant} |
| `.../src/middleware/oidc.rs` | create |
| `.../src/middleware/cedar.rs` | create |
| `.../src/middleware/spiffe.rs` | create |

### `oya-foundry-providers-router-worker`

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/health_monitor.rs` | create — per-provider rolling-window SLI scrape; emits to Mimir |
| `.../src/cost_rollup.rs` | create — per-tenant per-day cost roll-up; emits ceiling-breach events |
| `.../src/event_emitter.rs` | create — `ProviderInvoked` + `RouterDecided` + `CredentialResolved` to NATS |
| `.../src/demote_recover.rs` | create — drives provider-router demote/recover based on health |

### `oya-foundry-providers-router-app`

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/main.rs` | create — wires every crate via DI |
| `.../src/config.rs` | create — env + file config |
| `.../src/telemetry.rs` | create — OTel init |
| `.../src/signals.rs` | create — graceful shutdown |

## Test Plan

| Test | Verifies |
|---|---|
| `tests/integration/router_rest_decide_happy_path.rs` | REST end-to-end |
| `tests/integration/router_rest_oidc_unauthenticated_denied.rs` | OIDC middleware |
| `tests/integration/router_rest_cedar_cross_tenant_denied.rs` | Cedar middleware |
| `tests/integration/worker_health_monitor_demotes_on_unavailability` | health-monitor logic |
| `tests/integration/worker_cost_rollup_ceiling_breach_emits_event` | cost-rollup |
| `tests/load/router_decision.rs` | router decision p99 ≤ 5 ms over 100K decisions |
| `tests/integration/end_to_end_provider_invoke_emits_signed_envelope` | full path |

## Acceptance Gates

Standard + load test.

## Next IP

[`IP-015-router-sdk.md`](IP-015-router-sdk.md)

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `providers`-bounded-context slice for `IP-014: router-rest + router-worker + router-app (composition root)`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: tenant-scoped provider routing with OpenBao credential isolation and API/subscription adapters. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/intelligence/capabilities/providers-provider-route.yaml`, `microservices/intelligence/capabilities/providers-provider-invoke.yaml`, `microservices/intelligence/capabilities/providers-credential-resolve.yaml`, `microservices/intelligence/contracts/openapi/providers-provider-router.yaml`, and the policy set `microservices/intelligence/policy/providers-provider-router-tenant-scope.cedar`, `microservices/intelligence/policy/providers-openbao-credential.cedar`, `microservices/intelligence/policy/providers-credential-isolation.md`.

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
