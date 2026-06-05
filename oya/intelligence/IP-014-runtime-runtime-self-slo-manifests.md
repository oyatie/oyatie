---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-014-runtime-self-slo-manifests
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime + axis-observability
acceptance_lanes: [openslo-conformance, vcs-promotion-readiness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: Runtime self-SLO manifests (availability / latency / correctness / freshness)

## Intent

Author the four OpenSLO manifests under `microservices/intelligence/slos/` so the observability µservice's SLO engine can gate foundry-runtime's `dev → staging → production` promotion per ADR-0139. This IP is the gateway from "feature complete" to "production-promotable" — without these manifests + verdict=eligible, no promotion path past `dev` per `agentic-slo-gated-promotion.json`.

## ChangeSet boundary

Four OpenSLO YAML manifests + a one-line `slos/README.md` pointer. No Rust crate changes.

## Concrete File Targets

| Path | Action |
|---|---|
| `slos/availability.openslo.yaml` | create (target 99.95% over 30d) |
| `slos/latency.openslo.yaml` | create (p99 ≤ 50ms over 30d; the headline scalability target) |
| `slos/correctness.openslo.yaml` | create (zero autonomy-bypass + zero cross-tenant; 100% target) |
| `slos/freshness.openslo.yaml` | create (cache age ≤ 30s; 99.5% target) |
| `slos/README.md` | create (one-line pointer to PRD §"Performance Targets") |

(Note: manifests already created in this artifact pack; this IP is the formal claim + acceptance gate.)

## Acceptance Gates

```bash
# Schema-validate every manifest against OpenSLO v1.0
for slo in microservices/intelligence/slos/*.openslo.yaml; do
  cargo run -p oya-observability-slo-engine-rest -- validate $slo
done

# Verify slo-engine evaluator picks them up within hot-reload window
buck2 build //:quality-lane-registry-authority-check # lane=openslo-conformance --microservice foundry-runtime

# Verify initial verdict at staging tier
buck2 build //:quality-lane-registry-authority-check # lane=vcs-promotion-readiness --sha <head-sha> --env staging --microservice foundry-runtime
```

## Test Plan

| Test | Verifies |
|---|---|
| Manifest schema conformance | every file passes OpenSLO v1.0 validator |
| Reasonable thresholds (per `observability-slo.md`) | fast-burn ≤ 100% AND target ≥ 99% |
| PromQL feasibility | every indicator expression resolves against current Mimir tenant=oya-self |
| Initial verdict | once staging traffic flows, SLI accumulates over evaluator cadence; verdict transitions from `held` (bootstrap) to `eligible` |

## Halt Conditions

- Manifest sets unrealistic threshold (e.g., 99.999% with no measurement basis) — refactor to honest target.
- Indicator expression references non-existent metric — refactor.

## Next IP

[`IP-015-hg-fr-hyperscaler-gate-registration.md`](IP-015-hg-fr-hyperscaler-gate-registration.md)

## References

- `microservices/observability/PRD.md` (SLO engine consumes manifests).
- `microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md`.
- `docs/standards/observability-slo.md` (cross-cutting OpenSLO authoring rules).
- ADR-0139.
- `microservices/intelligence/PRD.md` §"Performance Targets" (source-of-truth thresholds).

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `runtime`-bounded-context slice for `IP-014: Runtime self-SLO manifests (availability / latency / correctness / freshness)`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: session-coherent hosted agent invocation without tenant-owned runtime infrastructure. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/intelligence/capabilities/runtime-capability-execute.yaml`, `microservices/intelligence/capabilities/runtime-session-create.yaml`, `microservices/intelligence/capabilities/runtime-session-resume.yaml`, `microservices/intelligence/contracts/openapi/runtime-foundry-runtime.yaml`, and the policy set `microservices/intelligence/policy/runtime-tenant-scope.cedar`, `microservices/intelligence/policy/runtime-runtime-isolation.md`, `microservices/intelligence/policy/runtime-ci-scope.cedar`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `runtime` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/intelligence/manifest.json`; the primary implementation anchor is `crates/oya-intelligence-api/src/lib.rs` plus the matching catalog records under `microservices/intelligence/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/intelligence/contracts/openapi/runtime-foundry-runtime.yaml`, `microservices/intelligence/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, and `microservices/intelligence/contracts/proto/runtime-foundry-runtime.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/intelligence/policy/runtime-tenant-scope.cedar`, `microservices/intelligence/policy/runtime-runtime-isolation.md`, `microservices/intelligence/policy/runtime-ci-scope.cedar`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/intelligence/slos/runtime-latency.openslo.yaml`, `microservices/intelligence/slos/runtime-availability.openslo.yaml`, `microservices/intelligence/slos/runtime-freshness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/intelligence/capabilities/runtime-capability-execute.yaml`, `microservices/intelligence/capabilities/runtime-session-create.yaml`, `microservices/intelligence/capabilities/runtime-session-resume.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

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
- Policy resolution against `microservices/intelligence/policy/runtime-tenant-scope.cedar`, `microservices/intelligence/policy/runtime-runtime-isolation.md`, `microservices/intelligence/policy/runtime-ci-scope.cedar`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/intelligence/slos/runtime-latency.openslo.yaml`, `microservices/intelligence/slos/runtime-availability.openslo.yaml`, `microservices/intelligence/slos/runtime-freshness.openslo.yaml`; no acceptance by line count alone.
- `buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/intelligence/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/intelligence/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| OpenAI Assistants threads/runs and AWS Bedrock Agents runtime | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
