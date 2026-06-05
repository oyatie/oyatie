---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-004-evidence-pack-builder-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence
acceptance_lanes: [cargo-clippy, cargo-doc, lean-port-location, lean-layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: Evidence-pack-builder kernel crate

## Intent

`oya-foundry-evidence-evidence-pack-builder-kernel`: port traits for SignalSource (runtime/eval/guardrails/supervisor) + AuditChainBridge + Postgres index writer + S3 blob staging. Layer = `kernel`.

## ChangeSet boundary

Single Rust crate. Pure types + traits.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/Cargo.toml` | create | edition=2024 |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/lib.rs` | create | re-export |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/entities/evidence_pack.rs` | create | `EvidencePack` entity (canonical schema) |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/entities/eval_verdict_at_invocation.rs` | create | join entity |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/entities/guardrail_decision.rs` | create | per-decision entity |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/entities/autonomy_level_decision.rs` | create | T0..T3 + rationale_hash |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/runtime_signal_source.rs` | create | `RuntimeSignalSourcePort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/eval_signal_source.rs` | create | `EvalSignalSourcePort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/guardrails_signal_source.rs` | create | `GuardrailsSignalSourcePort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/supervisor_signal_source.rs` | create | `SupervisorSignalSourcePort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/audit_chain_bridge.rs` | create | `AuditChainBridgePort` trait: `emit(pack) -> Result<AuditEventId, BridgeError>` |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/evidence_index_writer.rs` | create | `EvidenceIndexWriterPort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/ports/dead_letter_store.rs` | create | `DeadLetterStorePort` trait |
| `crates/oya-foundry-evidence-evidence-pack-builder-kernel/src/errors.rs` | create | `PackBuilderError` |
| `Cargo.toml` (workspace) | edit | register |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-evidence-pack-builder-kernel
cargo clippy -p oya-foundry-evidence-evidence-pack-builder-kernel -- -D warnings
cargo doc -p oya-foundry-evidence-evidence-pack-builder-kernel --no-deps
buck2 build //:quality-lane-registry-authority-check # lane=port-location --microservice foundry-evidence
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice foundry-evidence
```

## Halt Conditions

- Any project-internal import — block.
- `AuditChainBridgePort` exposes substrate-internal types (e.g., Merkle proofs) — block; substrate types only enter via SDK re-exports in adapter layer.

## Next IP

[`IP-005-evidence-pack-builder-domain.md`](IP-005-evidence-pack-builder-domain.md)

## References

- ADR-0105 + ADR-0056.
- ADR-0131 §"Substrate split" — kernel never depends on substrate internals.

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `evidence`-bounded-context slice for `IP-004: Evidence-pack-builder kernel crate`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: cryptographically sealed evidence packs for invocations, evals, guardrails, and regulator exports. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/intelligence/capabilities/evidence-evidence-pack-build.yaml`, `microservices/intelligence/capabilities/evidence-evidence-query.yaml`, `microservices/intelligence/capabilities/evidence-regulator-export.yaml`, `microservices/intelligence/contracts/openapi/evidence-foundry-evidence.yaml`, and the policy set `microservices/intelligence/policy/evidence-tenant-scope.cedar`, `microservices/intelligence/policy/evidence-regulator-export-scope.cedar`, `microservices/intelligence/policy/evidence-evidence-pack-integrity.md`.

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
