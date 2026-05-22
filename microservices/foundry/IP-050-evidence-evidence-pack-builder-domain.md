---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-005-evidence-pack-builder-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence
acceptance_lanes: [cargo-clippy, cargo-doc, lean-layer-correctness, property-based-tests]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: Evidence-pack-builder domain crate

## Intent

`oya-foundry-evidence-evidence-pack-builder-domain`: pack-schema construction logic + invariants + framework-profile builders. Pure functions. Layer = `domain`; imports own-BC `kernel` only.

## ChangeSet boundary

Single Rust crate. Pure compute (no I/O).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/Cargo.toml` | create | edition=2024; depends on own kernel only |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/lib.rs` | create | re-export |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/assembly/pack_envelope.rs` | create | `build_pack_envelope(invocation, eval, guardrails, supervisor) -> EvidencePack` |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/assembly/pack_payload_sha.rs` | create | `compute_pack_payload_sha(envelope) -> Sha256` — canonical CBOR encoding (deterministic) |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/assembly/partial_pack.rs` | create | partial-pack assembly with `missing_sources` |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/invariants/single_bind.rs` | create | EPI-02 invariant: pack assembled exactly once per `(invocation_id, attempt_no)` |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/invariants/eval_temporal_correctness.rs` | create | EPI-07 invariant: eval verdict current at invocation_ts |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/invariants/authority_cohesion.rs` | create | EPI-08 invariant: autonomy-ceiling decision is single-author from foundry-supervisor |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/eu_ai_act.rs` | create | EU AI Act profile field selector |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/hipaa.rs` | create | HIPAA profile |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/gdpr.rs` | create | GDPR profile |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/kr_pipa.rs` | create | KR PIPA profile |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/soc2.rs` | create | SOC 2 profile |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/iso_27001.rs` | create | ISO 27001 profile |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/tests/pack_sha_determinism.rs` | create | property: two assemblies of same inputs produce same sha |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/tests/eval_temporal_correctness.rs` | create | property: verdict applied is current at ts |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/tests/framework_profile_field_completeness.rs` | create | every profile produces all required fields for baseline inputs |
| `Cargo.toml` (workspace) | edit | register |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-evidence-pack-builder-domain
cargo nextest run -p oya-foundry-evidence-evidence-pack-builder-domain
cargo clippy -p oya-foundry-evidence-evidence-pack-builder-domain -- -D warnings
cargo doc -p oya-foundry-evidence-evidence-pack-builder-domain --no-deps
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice foundry-evidence
```

## Halt Conditions

- Pack-payload-sha non-determinism detected — block; CBOR encoding must canonicalise.
- Any framework profile fails field-completeness test against baseline — block; profile is the regulator-export contract.
- Authority-cohesion test fails (autonomy-ceiling sourced from anywhere other than foundry-supervisor) — block.

## Next IP

[`IP-006-evidence-pack-builder-usecase-and-adapters.md`](IP-006-evidence-pack-builder-usecase-and-adapters.md)

## References

- `microservices/foundry/policy/evidence-pack-integrity.md` EPI-02, EPI-03, EPI-07, EPI-08.
- ADR-0024 (eval-evidence integration).

## Wave 15 bespoke substance conversion

### A. Problem this IP closes
This IP is the `evidence`-bounded-context slice for `IP-005: Evidence-pack-builder domain crate`. The stamped version named a target but did not explain how the slice closes Foundry's product gap: cryptographically sealed evidence packs for invocations, evals, guardrails, and regulator exports. The concrete gap is traceability from the implementation plan to real Foundry surfaces: `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`, `microservices/foundry/contracts/openapi/evidence-foundry-evidence.yaml`, and the policy set `microservices/foundry/policy/evidence-tenant-scope.cedar`, `microservices/foundry/policy/evidence-regulator-export-scope.cedar`, `microservices/foundry/policy/evidence-evidence-pack-integrity.md`.

### B. Technical approach
Implement the slice as a Foundry-owned ChangeSet, not as generic platform plumbing. The design starts at the capability or contract boundary, keeps tenant and principal fields in the DTO/event shape, and routes state changes through the `evidence` policy envelope before any adapter call. The implementation must use existing catalog and crate naming from `microservices/foundry/manifest.json`; the primary implementation anchor is `crates/oya-foundry-evidence-domain/src/lib.rs` plus the matching catalog records under `microservices/foundry/catalog/`.

### C. Deliverables bound to real artifacts
- Update or create the exact crate/catalog files named by this IP; do not use `.../` placeholder paths in the final ChangeSet.
- Keep OpenAPI/AsyncAPI/proto parity across `microservices/foundry/contracts/openapi/evidence-foundry-evidence.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, and `microservices/foundry/contracts/proto/evidence-foundry-evidence.proto` when the slice exposes a wire surface.
- Bind authorization to `microservices/foundry/policy/evidence-tenant-scope.cedar`, `microservices/foundry/policy/evidence-regulator-export-scope.cedar`, `microservices/foundry/policy/evidence-evidence-pack-integrity.md`; if a required Cedar entity or action is absent, add it to the Foundry policy file in the same ChangeSet.
- Bind SLO evidence to `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`; this IP is incomplete if the acceptance path cannot point to an OpenSLO file or a documented N/A.
- Keep capability metadata aligned with `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml` so supervisor/runtime/evidence can reason about risk class and tenant availability.

### D. Implementation sequence
1. Read `microservices/foundry/PRD.md` and the `evidence` row in `microservices/foundry/manifest.json`; record the exact bounded-context crate names before editing.
2. Replace placeholder file targets with concrete paths under `crates/`, `microservices/foundry/catalog/`, `microservices/foundry/contracts/`, `microservices/foundry/policy/`, or `microservices/foundry/slos/`.
3. Add the domain/API fields required for `tenant_id`, `principal_id`, `home_cell`, `jurisdiction_code`, `audit_event_class`, and idempotency where this slice creates state or emits events.
4. Wire Cedar or documented policy checks before adapter calls, especially for high-risk capabilities such as `credential-resolve`, `regulator-export`, `engage-kill-switch`, and provider invocation.
5. Add contract, unit, and integration tests at the crate or contract paths named above; tests must assert at least one denial/failure path, not only the happy path.
6. Emit or validate SLO/audit evidence through the Foundry evidence path so the ChangeSet can be verified by `oya verify --ci-required` and the service-specific gates.

### E. Acceptance evidence
- `cargo test -p <changed-foundry-crate>` or the narrowest crate test covering `crates/oya-foundry-evidence-domain/src/lib.rs`.
- Contract parity for `microservices/foundry/contracts/openapi/evidence-foundry-evidence.yaml` and `microservices/foundry/contracts/proto/evidence-foundry-evidence.proto` when DTOs or handlers change.
- Policy resolution against `microservices/foundry/policy/evidence-tenant-scope.cedar`, `microservices/foundry/policy/evidence-regulator-export-scope.cedar`, `microservices/foundry/policy/evidence-evidence-pack-integrity.md`, including a tenant mismatch denial and a CI/synthetic principal allowance where applicable.
- SLO or dashboard linkage against `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`; no acceptance by line count alone.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice foundry` plus `git diff --check` before promotion.

### F. Evidence anchors
- `microservices/foundry/PRD.md` FR-X1..FR-X7 for the supervisor-runtime-guardrails-providers-evidence chain.
- `microservices/foundry/competitor-parity-matrix.md` for Foundry's comparison to AWS Bedrock, Google Vertex AI, Azure AI Foundry, Anthropic Console, OpenAI, Palantir AIP, and LangSmith/LangGraph.
- `docs/decisions/ADR-0136-foundry-as-single-microservice.md` and `docs/decisions/ADR-0137-foundry-bounded-contexts.md` for the one-product/many-BC boundary.
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` for the anti-stamp bar.

### G. Counterpart closure
| Counterpart | Gap closed by this IP |
|---|---|
| Palantir AIP audit evidence and ServiceNow GRC evidence export | Foundry lands the equivalent product capability while preserving Oyatie-specific tenant isolation, OpenBao/SPIFFE credential posture, Cedar enforcement, and evidence-chain verification. |
| Palantir AIP / Azure AI Foundry | The slice is promoted only with traceable contract, policy, SLO, and evidence artifacts rather than a prose-only launch checklist. |
