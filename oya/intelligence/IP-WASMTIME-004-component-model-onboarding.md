# IP-WASMTIME-004 — Component Model + WIT onboarding for foundry tools

> ADR anchor: ADR-0200.
> Owner: `oya-foundry`.
> Estimate: 4 days.

## Goal

Onboard new Foundry tools via WIT (WebAssembly Interface
Types) definitions that match the `oya:foundry/*` import
allowlist. Tool authors write WIT, compile to component-model
bytecode, register in the tool registry.

## Why this IP

WASI Preview 2 Component Model is the ABI per ADR-0200. WIT
is how upstream Wasmtime expresses the interface contract.
Without WIT onboarding, tool authors guess at the ABI and
end up with imports that fail at instantiation.

## Tasks

### 1. WIT canonical surface

- `oya:foundry/argv_read` — read tool input.
- `oya:foundry/stdout_write` — write tool output.
- `oya:foundry/log` — structured logging.
- `oya:foundry/capability_token_check` — verify the per-call
  capability token.

### 2. Tool packaging

- Build script template emits component-model bytecode.
- Registry validates bytecode against the WIT contract.

### 3. Tests

- Sample tool written in Rust + `wit-bindgen` + compiled to
  component-model.
- Sample tool in TinyGo (per Fastly community precedent).
- Sample tool in C via clang/wasi-libc.

### 4. Acceptance criteria

- A Rust + TinyGo + C tool all run end-to-end through the
  Foundry sandbox.
- Imports outside the WIT contract fail at instantiation.

## References

- ADR-0200.
- BytecodeAlliance Component Model spec.
- WIT canonical reference (upstream).

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `runtime`-bounded-context slice for `IP-WASMTIME-004 — Component Model + WIT onboarding for foundry tools`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: Wasm tool sandboxing with capability-token binding, metered execution, and WIT packaging discipline. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/intelligence/capabilities/runtime-capability-execute.yaml`, `microservices/intelligence/capabilities/runtime-session-create.yaml`, `microservices/intelligence/contracts/openapi/runtime-foundry-runtime.yaml`, and the policy set `microservices/intelligence/policy/runtime-tenant-scope.cedar`, `microservices/intelligence/policy/runtime-runtime-isolation.md`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `runtime` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/intelligence/manifest.json`; the primary implementation anchor is `crates/oya-intelligence-api/src/lib.rs` plus the matching catalog records under `microservices/intelligence/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/intelligence/contracts/openapi/runtime-foundry-runtime.yaml`, `microservices/intelligence/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, and `microservices/intelligence/contracts/proto/runtime-foundry-runtime.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/intelligence/policy/runtime-tenant-scope.cedar`, `microservices/intelligence/policy/runtime-runtime-isolation.md`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/intelligence/slos/runtime-latency.openslo.yaml`, `microservices/intelligence/slos/runtime-correctness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/intelligence/capabilities/runtime-capability-execute.yaml`, `microservices/intelligence/capabilities/runtime-session-create.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/intelligence/PRD.md` and the `runtime` row in `microservices/intelligence/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/intelligence/catalog/`, `microservices/intelligence/contracts/`, `microservices/intelligence/policy/`, or `microservices/intelligence/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-intelligence-api/src/lib.rs`.
- Contract parity for `microservices/intelligence/contracts/openapi/runtime-foundry-runtime.yaml` and `microservices/intelligence/contracts/proto/runtime-foundry-runtime.proto` when DTOs or handlers change.
- Policy resolution against `microservices/intelligence/policy/runtime-tenant-scope.cedar`, `microservices/intelligence/policy/runtime-runtime-isolation.md`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/intelligence/slos/runtime-latency.openslo.yaml`, `microservices/intelligence/slos/runtime-correctness.openslo.yaml`; no acceptance by line count alone.
- `buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/intelligence/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/intelligence/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| Cloudflare Workers sandbox limits, GitHub Actions token scoping, and OpenAI tool execution controls | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
