# IP-WASMTIME-002 — Capability-token binding for foundry-tool sandbox

> ADR anchor: ADR-0200, ADR-0136.
> Owner: `oya-foundry`.
> Estimate: 3 days.

## Goal

Generate, rotate, and verify per-(tenant, sandbox-class,
call-site) capability tokens for the `foundry-tool` sandbox
class. The kernel rejects invocations whose token does not
verify.

## Why this IP

Per ADR-0200 §"Security model — no ambient authority", every
WASM invocation carries a capability token. Foundry's tool
runner must mint, attach, and verify those tokens.

## Pre-conditions

- IP-WASMTIME-001 lands.
- OpenBao mount for capability tokens (per ADR-0173).

## Tasks

### 1. Token minting

- On tenant onboarding to Foundry: mint a per-(tenant,
  sandbox-class) root token.
- On tool registration: mint a per-(tenant, sandbox-class,
  call-site) derived token bound to the root.

### 2. Token rotation

- Per-tenant key cycle = 90 days.
- Overlap window: 14 days.

### 3. Token verification

- Kernel verifies token at invocation time by checking the
  HMAC-SHA256 of (tenant_id, sandbox_class, call_site)
  against the OpenBao-stored secret for the tenant's current
  cycle.

### 4. Audit

- Token mint, rotate, revoke events emit into ADR-0145 audit
  chain.

### 5. Tests

- Unit tests for the HMAC verification path.
- Integration tests for rotation overlap window.
- Negative test: token from a prior cycle rejected after
  overlap window closes.

## Failure modes

- OpenBao unavailable: token verification fails; tool runner
  rejects all WASM invocations until OpenBao recovers.

## Acceptance criteria

- 100% of `foundry-tool` invocations verify a token.
- Audit chain captures mint / rotate / revoke events.

## References

- ADR-0200 §"Capability tokens".
- ADR-0173 secrets storage.

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `runtime`-bounded-context slice for `IP-WASMTIME-002 — Capability-token binding for foundry-tool sandbox`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: Wasm tool sandboxing with capability-token binding, metered execution, and WIT packaging discipline. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/foundry/capabilities/runtime-capability-execute.yaml`, `microservices/foundry/capabilities/runtime-session-create.yaml`, `microservices/foundry/contracts/openapi/runtime-foundry-runtime.yaml`, and the policy set `microservices/foundry/policy/runtime-tenant-scope.cedar`, `microservices/foundry/policy/runtime-runtime-isolation.md`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `runtime` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/foundry/manifest.json`; the primary implementation anchor is `crates/oya-intelligence-api/src/lib.rs` plus the matching catalog records under `microservices/foundry/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/foundry/contracts/openapi/runtime-foundry-runtime.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, and `microservices/foundry/contracts/proto/runtime-foundry-runtime.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/foundry/policy/runtime-tenant-scope.cedar`, `microservices/foundry/policy/runtime-runtime-isolation.md`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/foundry/slos/runtime-latency.openslo.yaml`, `microservices/foundry/slos/runtime-correctness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/foundry/capabilities/runtime-capability-execute.yaml`, `microservices/foundry/capabilities/runtime-session-create.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/foundry/PRD.md` and the `runtime` row in `microservices/foundry/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/foundry/catalog/`, `microservices/foundry/contracts/`, `microservices/foundry/policy/`, or `microservices/foundry/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-intelligence-api/src/lib.rs`.
- Contract parity for `microservices/foundry/contracts/openapi/runtime-foundry-runtime.yaml` and `microservices/foundry/contracts/proto/runtime-foundry-runtime.proto` when DTOs or handlers change.
- Policy resolution against `microservices/foundry/policy/runtime-tenant-scope.cedar`, `microservices/foundry/policy/runtime-runtime-isolation.md`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/foundry/slos/runtime-latency.openslo.yaml`, `microservices/foundry/slos/runtime-correctness.openslo.yaml`; no acceptance by line count alone.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/foundry/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/foundry/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| Cloudflare Workers sandbox limits, GitHub Actions token scoping, and OpenAI tool execution controls | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
